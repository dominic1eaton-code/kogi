import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { PortfolioResourcesComponent } from './portfolio-resources.component';

describe('PortfolioResourcesComponent', () => {
  let component: PortfolioResourcesComponent;
  let fixture: ComponentFixture<PortfolioResourcesComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioResourcesComponent, RouterTestingModule]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioResourcesComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
