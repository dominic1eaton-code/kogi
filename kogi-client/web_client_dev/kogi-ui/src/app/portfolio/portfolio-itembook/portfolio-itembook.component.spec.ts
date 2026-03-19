import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { PortfolioItembookComponent } from './portfolio-itembook.component';

describe('PortfolioItembookComponent', () => {
  let component: PortfolioItembookComponent;
  let fixture: ComponentFixture<PortfolioItembookComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioItembookComponent, RouterTestingModule]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioItembookComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
