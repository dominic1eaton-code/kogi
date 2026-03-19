import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { PortfolioContentComponent } from './portfolio-content.component';

describe('PortfolioContentComponent', () => {
  let component: PortfolioContentComponent;
  let fixture: ComponentFixture<PortfolioContentComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioContentComponent, RouterTestingModule]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioContentComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
